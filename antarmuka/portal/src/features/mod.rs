//! Features module - Business logic and domain services

pub mod auth;
pub mod microfrontends;
pub mod oauth;
pub mod session_monitor;

#[cfg(test)]
mod auth_tests;

pub use auth::*;
pub use microfrontends::*;
pub use oauth::*;
