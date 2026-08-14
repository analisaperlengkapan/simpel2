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
//! ## Who may reach what
//!
//! Authentication proves *which workload* called; authorization decides what it
//! may reach. The peer's certificate common name is its policy name (Vault's
//! cert-auth model), and every secret handler evaluates the requested path
//! against that policy through [`secreton_core::services::policy::authorize`] —
//! the same decision function the REST middleware calls, so the two boundaries
//! cannot enforce different rules under one policy name.
//!
//! Pass the posture explicitly as [`GrpcAuthorization`]. On the insecure
//! listener there is no certificate, hence no principal to authorize, so the
//! opt-out is a named variant carrying its reason rather than a missing policy
//! source — "nothing is enforced here" is logged at startup instead of inferred.

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
pub use server::{GrpcAuthorization, SecretonGrpcService};
pub use tls::{
    GrpcTlsConfig, ListenerSecurity, allow_insecure_from_env, resolve_listener_security,
};
