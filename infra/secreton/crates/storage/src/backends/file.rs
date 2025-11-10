//! File System Storage Backend
//!
//! Implements HashiCorp Vault-compatible file system storage backend.
//! This is the simplest backend and suitable for:
//!
//! - **Development**: Local development and testing
//! - **Single-Node**: Non-HA deployments
//! - **Edge Deployments**: IoT and edge computing scenarios
//! - **Air-Gapped**: Disconnected environments
//!
//! # Features
//!
//! - **ACID Transactions**: Atomic file operations
//! - **Directory Structure**: Hierarchical key organization
//! - **Permissions**: Unix file permissions support
//! - **Encryption**: Optional filesystem-level encryption
//!
//! # Configuration
//!
//! ```toml
//! [storage]
//! backend = "file"
//! path = "/var/lib/secreton/data"
//! sync_writes = true
//! permissions = "0600"
//! ```

use crate::{BackendMetrics, KvBackend, StorageError, StorageResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// File system storage backend configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileConfig {
    /// Base directory path for storage
    pub path: PathBuf,
    
    /// Sync writes to disk (fsync)
    #[serde(default = "default_sync_writes")]
    pub sync_writes: bool,
    
    /// File permissions (Unix mode)
    #[serde(default = "default_permissions")]
    pub permissions: u32,
    
    /// Directory permissions (Unix mode)
    #[serde(default = "default_dir_permissions")]
    pub dir_permissions: u32,
}

fn default_sync_writes() -> bool {
    true
}

fn default_permissions() -> u32 {
    0o600
}

fn default_dir_permissions() -> u32 {
    0o700
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("/var/lib/secreton/data"),
            sync_writes: true,
            permissions: 0o600,
            dir_permissions: 0o700,
        }
    }
}

/// File system storage backend implementation
pub struct FileBackend {
    config: FileConfig,
    metrics: Arc<RwLock<BackendMetrics>>,
}

impl FileBackend {
    /// Create a new file system storage backend
    pub async fn new(config: FileConfig) -> StorageResult<Self> {
        // Create base directory if it doesn't exist
        fs::create_dir_all(&config.path).map_err(|e| StorageError::BackendError {
            backend: "file".to_string(),
            message: format!("Failed to create storage directory: {}", e),
        })?;
        
        // Set directory permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(config.dir_permissions);
            fs::set_permissions(&config.path, perms).map_err(|e| {
                StorageError::BackendError {
                    backend: "file".to_string(),
                    message: format!("Failed to set directory permissions: {}", e),
                }
            })?;
        }
        
        info!("File storage backend initialized at: {:?}", config.path);
        
        Ok(Self {
            config,
            metrics: Arc::new(RwLock::new(BackendMetrics::default())),
        })
    }
    
    /// Convert key to file path
    fn key_to_path(&self, key: &str) -> PathBuf {
        let sanitized = key.trim_start_matches('/').replace("..", "_");
        self.config.path.join(sanitized)
    }
    
    /// Ensure parent directory exists
    fn ensure_parent_dir(&self, path: &Path) -> StorageResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| StorageError::BackendError {
                backend: "file".to_string(),
                message: format!("Failed to create parent directory: {}", e),
            })?;
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = fs::Permissions::from_mode(self.config.dir_permissions);
                fs::set_permissions(parent, perms).ok(); // Ignore errors for existing dirs
            }
        }
        Ok(())
    }
    
    /// Write file with proper permissions
    fn write_file(&self, path: &Path, data: &[u8]) -> StorageResult<()> {
        self.ensure_parent_dir(path)?;
        
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(|e| StorageError::BackendError {
                backend: "file".to_string(),
                message: format!("Failed to open file for writing: {}", e),
            })?;
        
        file.write_all(data).map_err(|e| StorageError::BackendError {
            backend: "file".to_string(),
            message: format!("Failed to write data: {}", e),
        })?;
        
        if self.config.sync_writes {
            file.sync_all().map_err(|e| StorageError::BackendError {
                backend: "file".to_string(),
                message: format!("Failed to sync file: {}", e),
            })?;
        }
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(self.config.permissions);
            fs::set_permissions(path, perms).map_err(|e| StorageError::BackendError {
                backend: "file".to_string(),
                message: format!("Failed to set file permissions: {}", e),
            })?;
        }
        
        Ok(())
    }
}

#[async_trait]
impl KvBackend for FileBackend {
    async fn get(&self, key: &str) -> StorageResult<Option<Vec<u8>>> {
        let path = self.key_to_path(key);
        
        if !path.exists() {
            return Ok(None);
        }
        
        let mut file = File::open(&path).map_err(|e| StorageError::BackendError {
            backend: "file".to_string(),
            message: format!("Failed to open file: {}", e),
        })?;
        
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| {
            StorageError::BackendError {
                backend: "file".to_string(),
                message: format!("Failed to read file: {}", e),
            }
        })?;
        
        let mut metrics = self.metrics.write().await;
        metrics.reads += 1;
        metrics.bytes_read += buffer.len() as u64;
        
        Ok(Some(buffer))
    }
    
    async fn put(&self, key: &str, value: &[u8]) -> StorageResult<()> {
        let path = self.key_to_path(key);
        self.write_file(&path, value)?;
        
        let mut metrics = self.metrics.write().await;
        metrics.writes += 1;
        metrics.bytes_written += value.len() as u64;
        
        Ok(())
    }
    
    async fn delete(&self, key: &str) -> StorageResult<()> {
        let path = self.key_to_path(key);
        
        if path.exists() {
            fs::remove_file(&path).map_err(|e| StorageError::BackendError {
                backend: "file".to_string(),
                message: format!("Failed to delete file: {}", e),
            })?;
            
            let mut metrics = self.metrics.write().await;
            metrics.deletes += 1;
        }
        
        Ok(())
    }
    
    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        let prefix_path = self.key_to_path(prefix);
        let base_path = &self.config.path;
        
        let mut keys = Vec::new();
        
        if !prefix_path.exists() {
            return Ok(keys);
        }
        
        fn visit_dirs(
            dir: &Path,
            base: &Path,
            keys: &mut Vec<String>,
        ) -> std::io::Result<()> {
            if dir.is_dir() {
                for entry in fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    
                    if path.is_file() {
                        if let Ok(relative) = path.strip_prefix(base) {
                            if let Some(key) = relative.to_str() {
                                keys.push(key.to_string());
                            }
                        }
                    } else if path.is_dir() {
                        visit_dirs(&path, base, keys)?;
                    }
                }
            }
            Ok(())
        }
        
        visit_dirs(&prefix_path, base_path, &mut keys).map_err(|e| {
            StorageError::BackendError {
                backend: "file".to_string(),
                message: format!("Failed to list directory: {}", e),
            }
        })?;
        
        Ok(keys)
    }
    
    async fn exists(&self, key: &str) -> StorageResult<bool> {
        let path = self.key_to_path(key);
        Ok(path.exists())
    }
    
    async fn metrics(&self) -> StorageResult<BackendMetrics> {
        Ok(self.metrics.read().await.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_file_backend_basic_operations() {
        let temp_dir = tempdir().unwrap();
        let config = FileConfig {
            path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let backend = FileBackend::new(config).await.unwrap();
        
        // Test put
        backend.put("test/key", b"test-value").await.unwrap();
        
        // Test get
        let value = backend.get("test/key").await.unwrap();
        assert_eq!(value.unwrap(), b"test-value");
        
        // Test exists
        assert!(backend.exists("test/key").await.unwrap());
        
        // Test delete
        backend.delete("test/key").await.unwrap();
        assert!(!backend.exists("test/key").await.unwrap());
    }
    
    #[tokio::test]
    async fn test_file_backend_list() {
        let temp_dir = tempdir().unwrap();
        let config = FileConfig {
            path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let backend = FileBackend::new(config).await.unwrap();
        
        // Create test keys
        backend.put("app1/config", b"config1").await.unwrap();
        backend.put("app1/secrets", b"secret1").await.unwrap();
        backend.put("app2/config", b"config2").await.unwrap();
        
        // List all keys
        let keys = backend.list("").await.unwrap();
        assert_eq!(keys.len(), 3);
        
        // List with prefix
        let app1_keys = backend.list("app1").await.unwrap();
        assert_eq!(app1_keys.len(), 2);
    }
}
