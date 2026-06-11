//! # authenc-grpc
//!
//! gRPC service for Authenc identity provider.
//!
//! This crate provides service-to-service gRPC API for:
//! - Authentication (for backend services)
//! - Token validation
//! - User management
//! - mTLS enforcement

// Re-export types from authenc-types.
// Some names overlap the generated proto re-exports below (e.g. User); callers
// disambiguate via explicit paths, so the glob ambiguity is benign.
#[allow(ambiguous_glob_reexports)]
pub use authenc_types::*;

pub mod batch_operations;
pub mod captcha_service;
pub mod error;
pub mod health;
pub mod interceptors;
pub mod mfa_facade;
pub mod server;
pub mod service;
pub mod tls;

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
pub use batch_operations::{
    BatchPermissionResult, batch_check_permissions, batch_lookup_users, optimized_user_lookup,
};
pub use captcha_service::CaptchaGrpcService;
pub use health::{HealthService, ServingStatus, StandardHealthService};
pub use mfa_facade::MfaServiceFacadeImpl;
pub use proto::authenc::v1::{
    authenc_service_server::{AuthencService, AuthencServiceServer},
    *,
};
pub use server::{GrpcServerBuilder, GrpcServerConfig};
pub use service::{AuthencGrpcDeps, AuthencGrpcService, MfaServiceFacade, MfaSetupResponse};
pub use tls::TlsConfig;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
