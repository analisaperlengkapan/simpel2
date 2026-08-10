//! # Secreton gRPC Service
//!
//! This crate provides the gRPC API layer for Secreton engine.
//! Extracted from the main API crate for better modularity and compilation performance.
//!
//! ## Features
//! - Full gRPC service implementation
//! - mTLS with **enforced** client-certificate authentication ([`auth`])
//! - Protocol buffer definitions and generated code
//! - Integration with core engine services
//!
//! ## Bringing the listener up
//!
//! Call [`resolve_listener_security`] first. It fails closed: without mTLS
//! material the listener may only start when the operator set
//! `GRPC_ALLOW_INSECURE=true`. Then dispatch to
//! [`SecretonGrpcService::serve_with_mtls`] or, for the opt-out,
//! [`SecretonGrpcService::serve_insecure`].
//!
//! Authentication proves *which workload* called. It does not decide *what
//! that workload may reach* — that still needs policy evaluation on the secret
//! path (task #129).

// Generated proto code
pub mod generated {
    pub mod secreton {
        pub mod v1 {
            // Helper for generated code
            pub fn empty_body() -> http_body_util::Empty<bytes::Bytes> {
                http_body_util::Empty::new()
            }
            include!(concat!(env!("OUT_DIR"), "/secreton.v1.rs"));
        }
    }

    pub mod common {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/common.v1.rs"));
        }
    }
}

pub mod auth;
pub mod server;
pub mod tls;

pub use auth::{AllowedCommonNames, AnyTrustedPeer, ClientAuthLayer, PeerAuthorizer, PeerIdentity};
pub use server::SecretonGrpcService;
pub use tls::{
    GrpcTlsConfig, ListenerSecurity, allow_insecure_from_env, resolve_listener_security,
};
