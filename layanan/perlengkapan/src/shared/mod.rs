//! Cross-cutting infrastructure shared across every domain module.
//!
//! Per the refactor plan (Bagian A.3), most cross-cutting modules
//! (errors / database / cache / rate_limit / grpc_clients / health /
//! middleware / metrics / logging) currently live at the crate root
//! (`crate::errors`, `crate::database`, …). Moving them under
//! `crate::shared::*` is the structural cleanup the plan describes — to be
//! done as a focused PR with a single mechanical sweep of import paths.
//!
//! For now this module exposes only the pieces that need to land **before**
//! the full sweep can happen, because other modules depend on them:
//! - [`audit`] — concrete `PgAuditSink` implementing
//!   [`lib_perlengkapan::contracts::AuditSink`]. Plumbed into `AppState`
//!   and consumed by bantuan + notifikasi handlers to replace the
//!   `// TODO: Audit log` markers.

pub mod audit;
