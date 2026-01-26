pub mod audit;
pub mod auth;
pub mod config;
pub mod context;
pub mod correlation;
pub mod encoding;
pub mod error;
pub mod sanitizer;
pub mod validation;

// JWT claims module is always available (no crypto dependencies)
pub mod jwt_claims;

// Backend-only modules (require tokio runtime)
#[cfg(feature = "backend")]
pub mod cache;

#[cfg(feature = "backend")]
pub mod memory;

// Full JWT module with decode functionality (requires jsonwebtoken)
#[cfg(feature = "jwt")]
pub mod jwt;

#[cfg(feature = "telemetry")]
pub mod telemetry;

#[cfg(feature = "db")]
pub mod db;

#[cfg(feature = "crypto")]
pub mod crypto;

#[cfg(feature = "grpc")]
pub mod grpc;

pub mod health;

#[cfg(feature = "backend")]
pub mod middleware;

pub mod models;

pub use error::CommonError;
