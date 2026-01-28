use crate::config::AppConfig;
use crate::error::AppError;
use aes_gcm::aead::{Aead, NewAead};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::{Engine as _, engine::general_purpose};
use rand::RngCore;
use std::path::{Path, PathBuf};
use tokio::{fs, io::AsyncWriteExt};

pub struct StorageService {
    pub config: AppConfig,
    cipher: Aes256Gcm,
}

impl StorageService {
    pub fn new(config: AppConfig) -> Self {
        let key = Key::from_slice(config.encryption_key.as_bytes());
        let cipher = Aes256Gcm::new(key);
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

    pub async fn save_encrypted(&self, filename: &str, data: &[u8]) -> Result<(), AppError> {
        self.validate_extension(filename)?;
        self.validate_size(data.len() as u64)?;
        let nonce = Self::random_nonce();
        let ciphertext = self
            .cipher
            .encrypt(&nonce, data)
            .map_err(|_| AppError::Internal)?;
        let mut file = fs::File::create(self.storage_path(filename)).await?;
        file.write_all(&nonce).await?;
        file.write_all(&ciphertext).await?;
        Ok(())
    }

    pub async fn load_decrypted(&self, filename: &str) -> Result<Vec<u8>, AppError> {
        let path = self.storage_path(filename);
        let data = fs::read(&path).await?;
        if data.len() < 12 {
            return Err(AppError::Internal);
        }
        let (nonce, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce);
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| AppError::Internal)?;
        Ok(plaintext)
    }

    pub async fn delete(&self, filename: &str) -> Result<(), AppError> {
        let path = self.storage_path(filename);
        if fs::metadata(&path).await.is_ok() {
            fs::remove_file(&path).await?;
        }
        Ok(())
    }

    fn random_nonce() -> Nonce {
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);
        Nonce::from_slice(&nonce).clone()
    }
}
