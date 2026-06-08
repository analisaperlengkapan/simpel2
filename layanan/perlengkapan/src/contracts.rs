//! Cross-module trait contracts for the Perlengkapan service.
//!
//! Modules in the unified service crate depend on these traits rather than on
//! each other's concrete services. This is a "ports & adapters" boundary that
//! keeps `workflow` (the orchestrator) decoupled from `dokumen` and
//! `notifikasi` (the leaf services).
//!
//! Concrete impls live alongside in this crate:
//! - `DocumentGenerator` → `src/dokumen/service.rs`
//! - `NotificationSender` → `src/notifikasi/service.rs`
//! - `AuditSink` → `src/shared/audit.rs`
//! - `DocumentStorage` → `src/dokumen/filesystem_storage.rs`
//!
//! Moved out of `lib-perlengkapan` (F0-C): these traits use `async-trait` +
//! `bytes` and are only implemented/consumed here, so `lib-perlengkapan` stays
//! a pure WASM-safe DTO/domain crate. The DTOs the traits carry
//! (`AuditEvent`, `ServiceResult`) remain in `lib-perlengkapan`.

use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use lib_perlengkapan::audit::AuditEvent;
use lib_perlengkapan::error::ServiceResult;

// ---------------------------------------------------------------------------
// DocumentGenerator (workflow → dokumen)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentFormat {
    Pdf,
    Excel,
    /// Microsoft Word `.docx`. Generated alongside PDF for konsep surat / SK
    /// so users can edit before signing.
    Docx,
    Html,
    Csv,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentRequest {
    pub template_id: String,
    pub format: DocumentFormat,
    /// JSON-shaped data the template engine renders into.
    pub data: serde_json::Value,
    pub locale: Option<String>,
    pub requested_by: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email,
    Sms,
    Whatsapp,
    Push,
    InApp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub variables: Option<serde_json::Value>,
    /// Optional deeplink URL the in-app/push notification opens.
    pub deeplink: Option<String>,
    /// SMTP destination. Required when [`NotificationChannel::Email`] is in
    /// `channels`; the dispatcher skips the email channel with a log line
    /// when this is `None` rather than scanning the DB for a fallback. Keep
    /// this populated upstream wherever you already have the user record in
    /// hand (workflow engine, ticket service, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipient_email: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
