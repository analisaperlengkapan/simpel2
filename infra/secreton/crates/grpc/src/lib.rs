//! # Secreton gRPC Service
//!
//! This crate provides the gRPC API layer for Secreton vault.
//! Extracted from the main API crate for better modularity and compilation performance.
//!
//! ## Features
//! - Full gRPC service implementation
//! - mTLS support for secure communication
//! - Protocol buffer definitions and generated code
//! - Integration with core vault services

// Generated proto code from OUT_DIR
pub mod generated {
    /// Mewakili pub `secreton`.
    pub mod secreton {
        /// Mewakili pub `v1`.
        pub mod v1 {
            tonic::include_proto!("secreton.v1");
        }
    }

    /// Mewakili pub `common`.
    pub mod common {
        /// Mewakili pub `v1`.
        pub mod v1 {
            tonic::include_proto!("common.v1");
        }
    }
}

/// Mewakili pub `interceptor`.
pub mod interceptor;
/// Mewakili pub `server`.
pub mod server;
/// Mewakili pub `tls`.
pub mod tls;

pub use interceptor::{AuthConfig, auth_interceptor, extract_bearer_token};
pub use server::SecretonGrpcService;
pub use tls::GrpcTlsConfig;
