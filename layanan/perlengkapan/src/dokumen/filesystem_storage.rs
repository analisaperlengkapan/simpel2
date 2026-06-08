//! Filesystem-backed [`DocumentStorage`] adapter.
//!
//! Stores artifacts as plain files under a configurable root directory
//! (`DOCUMENT_STORAGE_PATH`, default `/tmp/perlengkapan/docs`). Designed so
//! the trait surface matches an eventual S3 implementation 1:1 —
//! `presigned_url` here just hands back the public HTTP route that
//! re-streams the file, but a future `S3Storage` will return an actual
//! S3-presigned URL through the same method, no caller changes needed.

use std::path::PathBuf;
use std::time::Duration;

use crate::contracts::{DocumentStorage, StorageHandle};
use async_trait::async_trait;
use chrono::Utc;
use lib_perlengkapan::ServiceError;
use sha2::{Digest, Sha256};

const DEFAULT_ROOT: &str = "/tmp/perlengkapan/docs";

/// Adapter implementing [`DocumentStorage`] on top of the local filesystem.
#[derive(Clone)]
pub struct FilesystemStorage {
    root: PathBuf,
    /// Public base URL the route handler streams from. Used to compose
    /// `presigned_url`; when `None`, a `file://` URI is returned so callers
    /// at least get *something* deterministic in test / CLI contexts.
    public_base: Option<String>,
}

impl FilesystemStorage {
    pub fn new(root: PathBuf, public_base: Option<String>) -> Self {
        Self { root, public_base }
    }

    pub fn from_env() -> Self {
        let root = std::env::var("DOCUMENT_STORAGE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_ROOT));
        let public_base = std::env::var("DOCUMENT_STORAGE_PUBLIC_BASE").ok();
        Self::new(root, public_base)
    }

    fn path_for(&self, key: &str) -> PathBuf {
        // The key is a path-relative segment like
        // `pemakaian-bmn/{id}/konsep-surat.pdf`; join under the root.
        // Reject `..` segments outright — callers control the key, but
        // defense in depth.
        let safe: PathBuf = key
            .split('/')
            .filter(|seg| !seg.is_empty() && *seg != ".." && *seg != ".")
            .collect();
        self.root.join(safe)
    }
}

#[async_trait]
impl DocumentStorage for FilesystemStorage {
    async fn put(
        &self,
        key: &str,
        bytes: bytes::Bytes,
        content_type: &str,
    ) -> Result<StorageHandle, ServiceError> {
        let path = self.path_for(key);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| ServiceError::storage(format!("mkdir {:?}: {}", parent, e)))?;
        }
        tokio::fs::write(&path, &bytes)
            .await
            .map_err(|e| ServiceError::storage(format!("write {:?}: {}", path, e)))?;

        // Cheap content hash for the etag — handy for client-side cache
        // validation. SHA256 is fast enough for the artifact sizes we
        // produce (PDF / DOCX / XLSX <= a few MB).
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let etag = format!("{:x}", hasher.finalize());

        Ok(StorageHandle {
            key: key.to_string(),
            size_bytes: bytes.len() as u64,
            content_type: content_type.to_string(),
            etag: Some(etag),
            stored_at: Utc::now(),
        })
    }

    async fn get(&self, key: &str) -> Result<bytes::Bytes, ServiceError> {
        let path = self.path_for(key);
        let data = tokio::fs::read(&path)
            .await
            .map_err(|e| ServiceError::storage(format!("read {:?}: {}", path, e)))?;
        Ok(bytes::Bytes::from(data))
    }

    async fn delete(&self, key: &str) -> Result<(), ServiceError> {
        let path = self.path_for(key);
        match tokio::fs::remove_file(&path).await {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(ServiceError::storage(format!("delete {:?}: {}", path, e))),
        }
    }

    async fn presigned_url(&self, key: &str, _ttl: Duration) -> Result<String, ServiceError> {
        // Filesystem impl can't sign — return the public-base URL pointing
        // at the route handler that streams `key`. The `ttl` is honoured
        // by the S3 implementation; here it's advisory only. Document for
        // operators in the env-var help block.
        Ok(match &self.public_base {
            Some(base) => format!("{}/{}", base.trim_end_matches('/'), key),
            None => format!("file://{}", self.path_for(key).display()),
        })
    }

    async fn exists(&self, key: &str) -> Result<bool, ServiceError> {
        let path = self.path_for(key);
        match tokio::fs::metadata(&path).await {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(ServiceError::storage(format!("stat {:?}: {}", path, e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> (FilesystemStorage, tempfile::TempDir) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let storage = FilesystemStorage::new(tmp.path().to_path_buf(), None);
        (storage, tmp)
    }

    #[tokio::test]
    async fn put_then_get_roundtrip() {
        let (storage, _tmp) = make();
        let payload = bytes::Bytes::from_static(b"hello world");
        let handle = storage
            .put("a/b/c.txt", payload.clone(), "text/plain")
            .await
            .expect("put");
        assert_eq!(handle.size_bytes, payload.len() as u64);
        assert!(handle.etag.is_some());
        let got = storage.get("a/b/c.txt").await.expect("get");
        assert_eq!(got, payload);
        assert!(storage.exists("a/b/c.txt").await.unwrap());
    }

    #[tokio::test]
    async fn delete_is_idempotent() {
        let (storage, _tmp) = make();
        storage.delete("nonexistent.txt").await.expect("idempotent");
        storage
            .put("x.txt", bytes::Bytes::from_static(b"x"), "text/plain")
            .await
            .unwrap();
        storage.delete("x.txt").await.expect("delete");
        assert!(!storage.exists("x.txt").await.unwrap());
    }

    #[tokio::test]
    async fn path_traversal_segments_are_stripped() {
        let (storage, tmp) = make();
        let _ = storage
            .put(
                "../../etc/evil.txt",
                bytes::Bytes::from_static(b"nope"),
                "text/plain",
            )
            .await
            .unwrap();
        // Should have landed at `<tmp>/etc/evil.txt`, NOT outside the root.
        let inside = tmp.path().join("etc/evil.txt");
        assert!(inside.exists());
    }
}
