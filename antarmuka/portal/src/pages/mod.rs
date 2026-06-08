//! Pages module — all page components
//!
//! Removes password_reset (admin-only in government context).

pub mod home;
pub mod not_found;

pub use home::*;
pub use not_found::*;
