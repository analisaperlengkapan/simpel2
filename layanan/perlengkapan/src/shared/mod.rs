//! Cross-cutting infrastructure shared across every domain module.
//!
//! Per the refactor plan (Bagian A.1 / A.3) the modules below were
//! collected here so domain code (`bank_aset`, `kebutuhan_bmn`, `dokumen`,
//! `notifikasi`, `bantuan`, …) imports its plumbing from one canonical
//! location instead of from a long flat list at the crate root. Each entry
//! corresponds to a section of the plan:
//!
//! - [`audit`] — concrete `PgAuditSink` impl of
//!   [`crate::contracts::AuditSink`].
//! - [`cache`] — cache manager wrapping deadpool-redis (`cache_strategy`
//!   originally).
//! - [`connection_config`] — db connection options helper.
//! - [`db`] — deadpool-postgres pool + `Database` wrapper.
//! - [`error`] — `AppError` + `AppResult` + `IntoResponse` mapping for
//!   the unified crate.
//! - [`grpc`] — backend↔backend clients (authenc + secreton + integrasi).
//! - [`health`] — `/health`, `/health/ready`, `/health/live` handlers.
//! - [`logging`] — `tracing-subscriber` init.
//! - [`metrics`] — Prometheus registry + helpers.
//! - [`middleware`] — JWT extractor (`Claims`), `ClientIp` extractor,
//!   prometheus middleware, RBAC helpers.
//! - [`rate_limit`] — token-bucket per-IP rate limiter middleware.

pub mod audit;
pub mod break_glass;
pub mod cache;
pub mod connection_config;
pub mod db;
pub mod error;
pub mod events;
pub mod grpc;
pub mod health;
pub mod logging;
pub mod metrics;
pub mod middleware;
pub mod pagination;
pub mod pdf;
pub mod pegawai_ref;
pub mod policy;
pub mod rate_limit;
pub mod repo;
pub mod resilience;
pub mod satker_scope;
pub mod search_db;
pub mod siman_columns;
pub mod status_tone;
pub mod upload;
