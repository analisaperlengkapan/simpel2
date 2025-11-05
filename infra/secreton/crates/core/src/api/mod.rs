//! API module for Secreton
//!
//! # ⚠️ DEPRECATED MODULE
//!
//! **This entire `api` module is deprecated and should not be used.**
//!
//! ## Why Deprecated?
//!
//! This module contains legacy Warp-based API handlers that were prototyped
//! during early development. The production API implementation has been moved
//! to the `secreton-api` crate, which uses:
//!
//! - **Axum** framework (more modern, better performance)
//! - **Proper error handling** with comprehensive ApiError types
//! - **Full middleware stack** (auth, rate limiting, CORS, etc.)
//! - **Integration with core services** without circular dependencies
//!
//! ## Migration Path
//!
//! Instead of importing from this module:
//!
//! ```rust,ignore
//! use secreton_core::api::{ApiError, ApiResponse, SecurityAPI};
//! ```
//!
//! Use the production API crate:
//!
//! ```rust,ignore
//! use secreton_api::{
//!     error::{ApiError, ApiResult},
//!     handlers,
//!     AppState,
//! };
//! ```
//!
//! ## See Also
//!
//! - Production API: `crates/api/src/`
//! - API documentation: `docs/layanan-api.md`
//! - Migration guide: `docs/API_MIGRATION_GUIDE.md` (if needed)

// Handlers module requires 'legacy-warp-api' feature (disabled by default)
#[cfg(feature = "legacy-warp-api")]
pub mod handlers;

#[cfg(feature = "legacy-warp-api")]
#[deprecated(
    since = "1.1.0",
    note = "This module contains legacy Warp-based handlers. Use `secreton-api` crate instead."
)]
pub use handlers::*;
