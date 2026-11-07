//! # Cross-module audit trail (`/audit`)
//!
//! BPK-ready reader over `perlengkapan.audit_log` — the canonical
//! cross-module sink written by every module via
//! [`crate::contracts::AuditSink`] (table V028, retention V037).
//!
//! This endpoint spans every audited module (bantuan, pakaian dinas, pemakaian,
//! penghapusan, workflow transitions, break-glass, ...), so an auditor can pull
//! the *complete* trail for any single resource with
//! `?entity=...&resource_id=...`. `GET /audit/facets` lists which modules,
//! entities and actions actually occur, for the admin viewer's dropdowns.
//!
//! Access is restricted to cross-satker roles; see [`handlers::list_audit_trail`].

pub mod handlers;
pub mod models;
pub mod repository;

pub use handlers::{list_audit_facets, list_audit_trail};
