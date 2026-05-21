//! Cross-module trait contracts for the Perlengkapan service.
//!
//! Modules in the unified service crate depend on these traits rather than on
//! each other's concrete services. This is a "ports & adapters" boundary that
//! keeps `workflow` (the orchestrator) decoupled from `dokumen` and
//! `notifikasi` (the leaf services).
//!
//! Concrete impls live in the service crate:
//! - `DocumentGenerator` → `src/dokumen/service.rs`
//! - `NotificationSender` → `src/notifikasi/service.rs`
//! - `AuditSink` → `src/shared/audit.rs`
//! - `DocumentStorage` → `src/dokumen/storage.rs` (FilesystemStorage now,
//!   S3Storage in a future PR).
//!
//! This module is gated on the `contracts` feature so the lib stays light for
//! WASM consumers.

#![cfg(feature = "contracts")]

use std::time::Duration;

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::audit::AuditEvent;
use crate::error::ServiceResult;

// ---------------------------------------------------------------------------
// DocumentGenerator (workflow → dokumen)
// ---------------------------------------------------------------------------

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentFormat {
    Pdf,
    Excel,
    /// Microsoft Word `.docx`. Generated alongside PDF for konsep surat / SK
    /// so users can edit before signing.
    Docx,
    Html,
    Csv,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentRequest {
    pub template_id: String,
    pub format: DocumentFormat,
    /// JSON-shaped data the template engine renders into.
    #[cfg(feature = "serde")]
    pub data: serde_json::Value,
    #[cfg(not(feature = "serde"))]
    pub data: String,
    pub locale: Option<String>,
    pub requested_by: Option<Uuid>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentArtifact {
    pub document_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: u64,
    /// Storage handle (key in object/filesystem storage). Use
    /// [`DocumentStorage::get`] to retrieve bytes.
    pub storage_key: String,
    pub generated_at: DateTime<Utc>,
}

#[async_trait::async_trait]
pub trait DocumentGenerator: Send + Sync {
    async fn generate(&self, request: DocumentRequest) -> ServiceResult<DocumentArtifact>;

    /// Preview a document without persisting it. Returns raw bytes (PDF/Excel
    /// blob) for the caller to stream to the browser.
    async fn preview(&self, request: DocumentRequest) -> ServiceResult<bytes::Bytes>;
}

// ---------------------------------------------------------------------------
// NotificationSender (workflow/bantuan → notifikasi)
// ---------------------------------------------------------------------------

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotificationChannel {
    Email,
    Sms,
    Whatsapp,
    Push,
    InApp,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NotificationPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationMessage {
    /// Logical event name (e.g. "workflow.approval_required",
    /// "tiket.dibuat"). Maps to a Handlebars template id.
    pub event: String,
    pub recipient_user_id: Uuid,
    pub channels: Vec<NotificationChannel>,
    pub priority: NotificationPriority,
    pub title: String,
    pub body: String,
    /// Optional template variables. Merged with user/system context before
    /// rendering.
    #[cfg(feature = "serde")]
    pub variables: Option<serde_json::Value>,
    #[cfg(not(feature = "serde"))]
    pub variables: Option<String>,
    /// Optional deeplink URL the in-app/push notification opens.
    pub deeplink: Option<String>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationReceipt {
    pub notification_id: Uuid,
    pub queued_at: DateTime<Utc>,
    pub channels_dispatched: Vec<NotificationChannel>,
}

#[async_trait::async_trait]
pub trait NotificationSender: Send + Sync {
    async fn send(&self, message: NotificationMessage) -> ServiceResult<NotificationReceipt>;
}

// ---------------------------------------------------------------------------
// AuditSink (every module → shared audit_log table)
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
pub trait AuditSink: Send + Sync {
    async fn log(&self, event: AuditEvent) -> ServiceResult<()>;
}

// ---------------------------------------------------------------------------
// DocumentStorage (dokumen module's storage backend)
// ---------------------------------------------------------------------------
//
// Abstraction designed so the filesystem implementation now can be swapped for
// S3-compatible storage in a follow-up PR without touching the dokumen module
// itself.

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageHandle {
    pub key: String,
    pub size_bytes: u64,
    pub content_type: String,
    pub etag: Option<String>,
    pub stored_at: DateTime<Utc>,
}

#[async_trait::async_trait]
pub trait DocumentStorage: Send + Sync {
    /// Store `bytes` under `key`. Returns a handle with metadata.
    async fn put(
        &self,
        key: &str,
        bytes: bytes::Bytes,
        content_type: &str,
    ) -> ServiceResult<StorageHandle>;

    /// Retrieve the bytes stored at `key`.
    async fn get(&self, key: &str) -> ServiceResult<bytes::Bytes>;

    /// Delete the object at `key`. Idempotent — no error if missing.
    async fn delete(&self, key: &str) -> ServiceResult<()>;

    /// Issue a (short-lived) URL that lets the browser download `key` directly
    /// without proxying through the service. Filesystem impl returns a
    /// signed-handler URL on the service itself; S3 impl will return a
    /// presigned S3 URL.
    async fn presigned_url(&self, key: &str, ttl: Duration) -> ServiceResult<String>;

    /// Whether the object exists.
    async fn exists(&self, key: &str) -> ServiceResult<bool>;
}
