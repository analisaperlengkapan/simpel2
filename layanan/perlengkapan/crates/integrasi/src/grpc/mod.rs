//! gRPC module for layanan-integrasi
//!
//! This module provides gRPC server and client implementations for the
//! integration service, enabling other services (like layanan-perlengkapan)
//! to access MonSAKTI, MySIMKARI, and SIMAN data.

#[cfg(feature = "grpc")]
pub mod proto {
    //! Generated protobuf code
    tonic::include_proto!("integrasi.v1");
}

#[cfg(feature = "grpc")]
pub mod server;

#[cfg(feature = "grpc")]
pub mod service;

#[cfg(feature = "grpc")]
pub use proto::integrasi_service_server::IntegrasiServiceServer;

#[cfg(feature = "grpc")]
pub use server::start_grpc_server;

#[cfg(feature = "grpc")]
pub use service::IntegrasiServiceImpl;
