//! Concrete service implementations injected into `ApiState`.
//!
//! `mfa_store` — Postgres backings for `authenc_mfa::TotpStore` and
//! `BackupCodesStore`. Lives here (not in `authenc-storage`) so we can
//! depend on `authenc-mfa` without inverting the existing dependency
//! graph (`authenc-core` → `authenc-storage` already, and `authenc-core`
//! depends on `authenc-mfa`, so storage cannot reach for mfa).
//!
//! `mfa_api` — `LocalMfaApi`, the `MfaApiService` adapter that drives the
//! REST handlers in `handlers::mfa`.

pub mod mfa_api;
pub mod mfa_store;

pub use mfa_api::LocalMfaApi;
pub use mfa_store::{PgBackupCodesStore, PgTotpStore};
