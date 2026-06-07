//! Features module - Business logic and domain services
//!
//! Feature-first layout (F0-B): each bounded context owns a directory.
//! - `auth/`     — auth context/service + OAuth flow + tests
//! - `session/`  — cross-tab session monitor
//! - `microfrontends` — cross-cutting microfrontend registry

pub mod auth;
pub mod microfrontends;
pub mod session;

pub use auth::*;
pub use microfrontends::*;
