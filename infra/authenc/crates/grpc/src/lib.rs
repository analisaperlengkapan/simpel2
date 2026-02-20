//! # authenc-grpc
//!
//! gRPC service for Authenc identity provider.
//!
//! This crate provides service-to-service gRPC API for:
//! - Authentication (for backend services)
//! - Token validation
//! - User management
//! - mTLS enforcement

// Re-export types from authenc-types
pub use authenc_types::*;

pub mod service;
pub mod interceptors;
pub mod tls;
pub mod error;
pub mod server;
pub mod mfa_facade;

// Generated proto code
pub mod proto {
    pub mod authenc {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/authenc.v1.rs"));
        }
    }
    pub mod common {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/common.v1.rs"));
        }
    }
}

// Re-export commonly used types
pub use proto::authenc::v1::{
    authenc_service_server::{AuthencService, AuthencServiceServer},
    *,
};
pub use server::{GrpcServerBuilder, GrpcServerConfig};
pub use service::{AuthencGrpcService, MfaServiceFacade, MfaSetupResponse};
pub use mfa_facade::MfaServiceFacadeImpl;
pub use tls::TlsConfig;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
