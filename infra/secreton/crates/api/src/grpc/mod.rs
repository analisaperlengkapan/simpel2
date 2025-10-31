//! gRPC API module for Secreton
//!
//! This module provides gRPC service implementation using the proto definitions
//! from infra/proto/secreton.proto

// Generated proto code
pub mod secreton {
    pub mod v1 {
        include!("generated/secreton.v1.rs");
    }
}

pub mod common {
    pub mod v1 {
        include!("generated/common.v1.rs");
    }
}

pub mod server;
pub mod tls;

pub use server::SecretonGrpcService;
pub use tls::GrpcTlsConfig;
