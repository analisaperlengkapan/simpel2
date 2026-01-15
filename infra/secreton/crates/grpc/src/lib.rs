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
    pub mod secreton {
        pub mod v1 {
            tonic::include_proto!("secreton.v1");
        }
    }

    pub mod common {
        pub mod v1 {
            tonic::include_proto!("common.v1");
        }
    }
}

pub mod interceptor;
pub mod server;
pub mod tls;

pub use interceptor::{AuthConfig, auth_interceptor, extract_bearer_token};
pub use server::SecretonGrpcService;
pub use tls::GrpcTlsConfig;
