use crate::config::AppConfig;
use crate::error::AppError;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::{fs, io::AsyncWriteExt};
use uuid::Uuid;

pub struct StorageService {
    pub config: AppConfig,
    cipher: Aes256Gcm,
}

impl StorageService {
    pub fn new(config: AppConfig) -> Self {
        let key_bytes = config.encryption_key.as_bytes();
        let cipher = Aes256Gcm::new_from_slice(key_bytes)
            .expect("Encryption key must be exactly 32 bytes for AES-256");
        Self { config, cipher }
    }

    pub fn validate_extension(&self, filename: &str) -> Result<(), AppError> {
        let ext = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if self.config.allowed_extensions.iter().any(|e| e == &ext) {
            Ok(())
        } else {
            Err(AppError::Validation(format!(
                "Ekstensi {} tidak diizinkan",
                ext
            )))
        }
    }

    pub fn validate_size(&self, size: u64) -> Result<(), AppError> {
        if size > self.config.max_file_size {
            Err(AppError::Validation(format!(
                "Ukuran file {} melebihi batas {}",
                size, self.config.max_file_size
            )))
        } else {
            Ok(())
        }
    }

    pub fn storage_path(&self, filename: &str) -> PathBuf {
        let mut path = PathBuf::from(&self.config.storage_path);
        path.push(filename);
        path
    }

    /// Calculate SHA-256 checksum of data
    pub fn calculate_checksum(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Verify checksum of data
    pub fn verify_checksum(&self, data: &[u8], expected_checksum: &str) -> Result<(), AppError> {
        let actual_checksum = self.calculate_checksum(data);
        if actual_checksum == expected_checksum {
            Ok(())
        } else {
            Err(AppError::Validation(format!(
                "Checksum mismatch: expected {}, got {}",
                expected_checksum, actual_checksum
            )))
        }
    }

    /// Save encrypted file with checksum
    pub async fn save_encrypted(&self, filename: &str, data: &[u8]) -> Result<String, AppError> {
        // Calculate checksum before encryption
        let checksum = self.calculate_checksum(data);

        let nonce_bytes = Self::random_nonce_bytes();
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = self
            .cipher
            .encrypt(nonce, data)
            .map_err(|_| AppError::Internal)?;

        // Ensure parent directory exists
        let path = self.storage_path(filename);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut file = fs::File::create(&path).await?;
        file.write_all(&nonce_bytes).await?;
        file.write_all(&ciphertext).await?;

        Ok(checksum)
    }

    /// Load and decrypt file with checksum verification
    pub async fn load_decrypted(&self, filename: &str) -> Result<Vec<u8>, AppError> {
        let path = self.storage_path(filename);
        let data = fs::read(&path).await?;
        if data.len() < 12 {
            return Err(AppError::Internal);
        }
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| AppError::Internal)?;
        Ok(plaintext)
    }

    /// Save document version
    pub async fn save_version(
        &self,
        pool: &deadpool_postgres::Pool,
        document_id: Uuid,
        version: i32,
        data: &[u8],
    ) -> Result<String, AppError> {
        let version_filename = format!("{}_v{}", document_id, version);
        let checksum = self.save_encrypted(&version_filename, data).await?;

        let client = pool.get().await?;
        client
            .execute(
                "INSERT INTO dokumen.document_versions (document_id, version, storage_path, size, checksum, encrypted)
                 VALUES ($1, $2, $3, $4, $5, true)",
                &[&document_id, &version, &version_filename, &(data.len() as i64), &checksum],
            )
            .await?;

        Ok(checksum)
    }

    /// Get document version
    pub async fn get_version(
        &self,
        pool: &deadpool_postgres::Pool,
        document_id: Uuid,
        version: i32,
    ) -> Result<Vec<u8>, AppError> {
        let client = pool.get().await?;
        let row = client
            .query_opt(
                "SELECT storage_path, checksum FROM dokumen.document_versions WHERE document_id = $1 AND version = $2",
                &[&document_id, &version],
            )
            .await?
            .ok_or_else(|| AppError::NotFound("Version not found".to_string()))?;

        let storage_path: String = row.get("storage_path");
        let checksum: Option<String> = row.get("checksum");

        let data = self.load_decrypted(&storage_path).await?;

        if let Some(checksum) = checksum {
            self.verify_checksum(&data, &checksum)?;
        }

        Ok(data)
    }

    /// Search documents by filename or metadata
    pub async fn search_documents(
        &self,
        pool: &deadpool_postgres::Pool,
        query: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<crate::models::Document>, AppError> {
        let search_pattern = format!("%{}%", query);
        let client = pool.get().await?;
        let rows = client
            .query(
                "SELECT * FROM dokumen.documents WHERE filename ILIKE $1 OR metadata::text ILIKE $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
                &[&search_pattern, &limit, &offset],
            )
            .await?;

        Ok(rows.iter().map(crate::models::Document::from).collect())
    }

    /// List all versions of a document
    pub async fn list_versions(
        &self,
        pool: &deadpool_postgres::Pool,
        document_id: Uuid,
    ) -> Result<Vec<crate::models::DocumentVersion>, AppError> {
        let client = pool.get().await?;
        let rows = client
            .query(
                "SELECT * FROM dokumen.document_versions WHERE document_id = $1 ORDER BY version DESC",
                &[&document_id],
            )
            .await?;

        Ok(rows
            .iter()
            .map(crate::models::DocumentVersion::from)
            .collect())
    }

    pub async fn delete(&self, filename: &str) -> Result<(), AppError> {
        let path = self.storage_path(filename);
        if fs::metadata(&path).await.is_ok() {
            fs::remove_file(&path).await?;
        }
        Ok(())
    }

    fn random_nonce_bytes() -> [u8; 12] {
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);
        nonce
    }
}
