//! # Cross-module audit trail (`/audit`)
//!
//! BPK-ready reader over `perlengkapan.audit_log` — the canonical
//! cross-module sink written by every module via
//! [`lib_perlengkapan::contracts::AuditSink`] (table V028, retention V037).
//!
//! Distinct from `/admin/audit`, which surfaces only workflow transitions
//! (`workflow_transitions`). This endpoint spans login/export/document/
//! notifikasi/workflow events alike, so an auditor can pull the *complete*
//! trail for any single resource with `?entity=...&resource_id=...`.
//!
//! Access is restricted to cross-satker roles; see [`handlers::list_audit_trail`].

pub mod handlers;
pub mod models;
pub mod repository;

pub use handlers::list_audit_trail;
