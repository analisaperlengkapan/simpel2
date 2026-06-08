//! Features module - Business logic and domain services
//!
//! Feature-first layout (F0-B): each bounded context owns a directory.
//! - `auth/`     — auth context/service + OAuth flow + login/callback pages
//! - `mfa/`      — TOTP setup/verification + backup codes
//! - `session/`  — cross-tab session monitor
//! - `microfrontends` — cross-cutting microfrontend registry

pub mod auth;
pub mod mfa;
pub mod microfrontends;
pub mod notifications;
pub mod session;

pub use auth::*;
pub use microfrontends::*;
