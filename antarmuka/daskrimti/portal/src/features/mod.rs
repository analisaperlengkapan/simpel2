//! Features module - Business logic and domain services

pub mod auth;
pub mod microfrontends;
pub mod oauth;

#[cfg(test)]
mod auth_tests;

pub use auth::*;
pub use microfrontends::*;
pub use oauth::*;
