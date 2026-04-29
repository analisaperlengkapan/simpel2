//! Typed route enums shared across `antarmuka/portal` and
//! `antarmuka/perlengkapan`.
//!
//! Replaces the parallel `mod segment { ... }` / `mod path { ... }` /
//! `mod url { ... }` constant blocks scattered across each app crate.
//! One enum per app collapses all three: variants are the unit
//! (parameter-free) routes, tuple variants carry path parameters,
//! and `to_path()` returns the final URL string.
//!
//! ```ignore
//! use lib_ui::routes::PerlengkapanRoute;
//! let url = PerlengkapanRoute::KebutuhanDetail("abc".into()).to_path();
//! assert_eq!(url, "/perlengkapan/kebutuhan-bmn/detail/abc");
//! ```
//!
//! This module is intentionally pure data — no Leptos macros, no
//! components — so call-sites can adopt it incrementally without a
//! flag-day migration of `<A href=routes::path::DASHBOARD>` chains.
//! The matching `<AppLink route=… />` ergonomic wrapper lands once
//! the workspace build is healthy enough to verify it.

pub mod perlengkapan;
pub mod portal;

pub use perlengkapan::PerlengkapanRoute;
pub use portal::PortalRoute;

/// Shared API: every typed route enum can produce its full path.
pub trait ToPath {
    fn to_path(&self) -> String;
}
